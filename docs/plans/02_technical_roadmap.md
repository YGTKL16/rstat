# rstat — Faz 2-5 Detaylı Teknik İmplementasyon Planı

> **Oluşturulma tarihi:** 2026-06-18
> **Temel:** Faz 0 ✅ + Faz 1 (50/50 test) ✅ tamamlandı.
> **Hedef:** Senior developer gözüyle, implementasyona doğrudan girilecek seviyede plan.

---

## 1. Mevcut Kod Tabanı Analizi

### 1.1 Modül Envanteri

| Dosya | Durum | Satır | Açıklama |
|---|---|---|---|
| `rstat-core/src/error.rs` | ✅ Sağlam | 20 | `StatError` enum — 5 varyant yeterli, genişletme gereksiz |
| `rstat-core/src/result.rs` | ⚠️ Büyüyecek | 39 | `SummaryStats`, `TTestResult`, `GroupStats` — yeni result struct'lar eklenecek |
| `rstat-core/src/data/summary.rs` | ✅ Sağlam | 112 | Welford variance, quantile, mean — her yerde reuse edilebilir |
| `rstat-core/src/dist/pvalue.rs` | ✅ Sağlam | 150 | `p_value()`, `ci_bounds()`, `critical_t()`, `Alternative` enum |
| `rstat-core/src/tests_stat/ttest.rs` | ✅ Referans pattern | 381 | `describe()` yardımcı fn + 3 pub fn + unit+proptest — **şablon** |
| `rstat-cli/src/cli.rs` | ⚠️ Büyüyecek | 73 | `Commands` enum'a yeni subcommand'ler eklenecek |
| `rstat-cli/src/io.rs` | ✅ + Eksik | 128 | `read_column`, `read_two_columns` var; `read_matrix` (chi-square için) eksik |
| `rstat-cli/src/render.rs` | ⚠️ Büyüyecek | 125 | `print_ttest`, `print_summary`; her komut için yeni print fn gerekecek |
| `rstat-cli/src/commands/ttest.rs` | ✅ Referans pattern | 39 | Argüman → core fn → render üçlüsü — **şablon** |

### 1.2 Güçlü Yönler (Devam Edilecek)

1. **`describe()` helper pattern** — `ttest.rs:22-26` satırları: `(n, mean, variance, std_dev)` tuple döner. ANOVA grup hesaplamalarında aynı pattern kullanılacak.

2. **`ci_bounds()` API tasarımı** — `est, se, df, ci_level, alt` parametresi. ANOVA sonrası Tukey CI için de uyarlanabilir.

3. **`Alternative` enum + `parse()`** — Hem ANOVA (F-testi yönlendirme yok ama Tukey CI'da gerekli) hem CI komutunda aynı enum reuse edilecek.

4. **`OutputFormat::detect()`** — Tüm yeni komutlar bu TTY/pipe otomasyonundan ücretsiz yararlanır.

5. **Test mimarisi** — `#[cfg(test)]` birim + `prop_tests` modülü + `cli_X.rs` entegrasyon; her faz için aynı 3 katman uygulanacak.

6. **`read_column` NA/NaN skip** — Mevcut implementasyon tüm sayısal komutlar için reuse edilebilir.

### 1.3 Teknik Borçlar

| Borç | Etki | Faz | Önerilen Çözüm |
|---|---|---|---|
| `render.rs` monolitik büyüme | Yüksek | Faz 2 başında | `render/` dizine bölün |
| `io.rs` içinde `run_summary` var | Orta | Faz 2 | `commands/summary.rs`'e taşı |
| `result.rs` tek dosya | Orta | Faz 3 sonrası | Modüle bölün |
| `atty = "0.2"` deprecated | Düşük | Faz 6 | `is-terminal` crate ile swap |
| Infinity input kabul ediliyor | Orta | Faz 2 | `read_column`'da `is_finite()` kontrolü ekle |

### 1.4 `statrs` Kapasitesi (Faz 2-5)

```rust
// Faz 2 — ANOVA
use statrs::distribution::{FisherSnedecor, ContinuousCDF};
let f_dist = FisherSnedecor::new(df_between, df_within)?;
let p = f_dist.sf(f_stat);  // 1.0 - CDF(F)

// Faz 2 — CI proportion
use statrs::distribution::Normal;
let z_dist = Normal::new(0.0, 1.0)?;
let z = z_dist.inverse_cdf(1.0 - alpha / 2.0);

// Faz 3 — Chi-square
use statrs::distribution::ChiSquared;
let chi_dist = ChiSquared::new(df)?;
let p = chi_dist.sf(chi_sq);

// Faz 5 — Capability PPM
let norm = Normal::new(mean, sigma)?;
let ppm = (norm.sf(usl) + norm.cdf(lsl)) * 1_000_000.0;
```

**statrs zaten Cargo.toml'da mevcut — hiç yeni bağımlılık gerekmez.**

---

## 2. Faz 2: ANOVA + Güven Aralıkları

### 2.1 Tahmini Süre: 5-6 iş günü

### 2.2 Modül 1: `rstat-core/src/tests_stat/anova.rs` (SIFIRDAN)

#### Struct Tanımları (`result.rs`'e eklenecek)

```rust
#[derive(Debug, Serialize)]
pub struct AnovaResult {
    pub test: &'static str,          // "one-way-anova"
    pub groups: Vec<GroupStats>,     // mevcut GroupStats reuse!
    // SS decomposition
    pub ss_between: f64,
    pub ss_within: f64,
    pub ss_total: f64,
    // Degrees of freedom
    pub df_between: f64,
    pub df_within: f64,
    // Mean squares
    pub ms_between: f64,
    pub ms_within: f64,
    // Test istatistiği
    pub f_statistic: f64,
    pub p_value: f64,
    // Etki büyüklüğü
    pub eta_squared: f64,            // SS_between / SS_total
    pub omega_squared: f64,          // (SS_b - df_b*MS_w) / (SS_t + MS_w)
    // Karar
    pub alpha: f64,
    pub reject_null: bool,
}
```

#### Fonksiyon İmzaları

```rust
/// One-way ANOVA.
/// `groups`: Her grup için &[f64] — minimum 2 grup, her grupta min 2 gözlem.
pub fn one_way(
    groups: &[&[f64]],
    alpha: f64,
) -> Result<AnovaResult, StatError>
```

#### Algoritma

```
Grand mean (x̄̄) = Σ(nᵢ * x̄ᵢ) / N

SS_between = Σ nᵢ * (x̄ᵢ - x̄̄)²    df_between = k - 1
SS_within  = Σᵢ Σⱼ (xᵢⱼ - x̄ᵢ)²   df_within  = N - k
SS_total   = SS_between + SS_within

MS_between = SS_between / df_between
MS_within  = SS_within  / df_within

F = MS_between / MS_within
p = FisherSnedecor(df_between, df_within).sf(F)

η² = SS_between / SS_total
ω² = (SS_between - df_between * MS_within) / (SS_total + MS_within)
```

#### Edge Case'ler

```rust
// 1. Tek grup → hata
if groups.len() < 2 {
    return Err(StatError::InsufficientData { required: 2, got: groups.len() });
}
// 2. MS_within = 0 (tüm grup içi varyans sıfır)
if ms_within == 0.0 {
    return Err(StatError::Numerical("MS_within sıfır".into()));
}
// 3. Dengesiz gruplar — formül hâlâ geçerli, uyarı gerekmez
```

#### Test Stratejisi

```rust
// scipy.stats.f_oneway([2,3,4], [5,6,7], [8,9,10]) → F=27.0
const G1: &[f64] = &[2.0, 3.0, 4.0];
const G2: &[f64] = &[5.0, 6.0, 7.0];
const G3: &[f64] = &[8.0, 9.0, 10.0];

#[test]
fn test_one_way_three_groups() {
    let r = one_way(&[G1, G2, G3], 0.05).unwrap();
    assert!((r.f_statistic - 27.0).abs() < 1e-9);
    assert!(r.reject_null);
}

// Property: F >= 0 her zaman
// Property: SS_total = SS_between + SS_within  (< 1e-10 tolerans)
// Property: p ∈ [0, 1]
// Property: η² ∈ [0, 1]
```

#### `scripts/crossvalidate_anova.py`

```python
from scipy import stats
import json

datasets = {
    "three_equal":  [[2,3,4], [5,6,7], [8,9,10]],
    "four_balanced":[[1,2,3,4,5],[2,3,4,5,6],[3,4,5,6,7],[4,5,6,7,8]],
    "unbalanced":   [[1,2,3],[4,5,6,7,8],[9,10]],
    "no_effect":    [[1,2,3,4,5],[2,3,4,5,6]],
}
results = {}
for name, groups in datasets.items():
    f, p = stats.f_oneway(*groups)
    results[name] = {"f_statistic": f, "p_value": p,
                     "df_between": len(groups)-1,
                     "df_within": sum(len(g) for g in groups)-len(groups)}
print(json.dumps(results, indent=2))
```

---

### 2.3 Modül 2: `rstat-core/src/interval/ci.rs` (SIFIRDAN, ama büyük ölçüde REUSE)

#### Struct Tanımları

```rust
#[derive(Debug, Serialize)]
pub struct CiResult {
    pub ci_type: &'static str,   // "mean" | "proportion" | "variance"
    pub method: &'static str,    // "t" | "wilson" | "wald" | "chi2"
    pub level: f64,
    pub lower: f64,
    pub upper: f64,
    pub point_estimate: f64,     // x̄, p̂, veya s²
    pub n: usize,
    pub std_err: Option<f64>,
    pub successes: Option<usize>,
    pub trials: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProportionMethod {
    Wilson,  // Varsayılan
    Wald,
}
```

#### Fonksiyon İmzaları

```rust
/// t-tabanlı mean CI — ci_bounds() reuse ile ~10 satır!
pub fn mean_ci(data: &[f64], level: f64) -> Result<CiResult, StatError>

/// Wilson veya Wald proportion CI
pub fn proportion_ci(
    successes: usize,
    trials: usize,
    level: f64,
    method: ProportionMethod,
) -> Result<CiResult, StatError>

/// Chi-square tabanlı variance CI
pub fn variance_ci(data: &[f64], level: f64) -> Result<CiResult, StatError>
```

#### Algoritmalar

```rust
// MEAN: ci_bounds() zaten tüm işi yapıyor!
// mean_ci() = ci_bounds(xbar, se=s/√n, df=n-1, level, TwoSided) çağrısı

// WILSON:
// z = Normal(0,1).ppf(1 - α/2)
// center = (k + z²/2) / (n + z²)
// margin = z * sqrt(p̂*(1-p̂)/n + z²/(4n²)) / (1 + z²/n)
// [center-margin, center+margin]

// VARIANCE (chi-square):
// lower = (n-1)*s² / χ²_{1-α/2, n-1}
// upper = (n-1)*s² / χ²_{α/2, n-1}
// statrs::ChiSquared::new(n-1)?.inverse_cdf(...)
```

---

### 2.4 CLI Katmanı — Yeni Dosyalar

#### `cli.rs`'e eklenecek struct'lar

```rust
#[derive(Parser)]
pub struct AnovaArgs {
    pub file: Option<PathBuf>,
    #[arg(long)] pub value: Option<String>,  // uzun format
    #[arg(long)] pub group: Option<String>,  // uzun format
    #[arg(long)] pub cols: Option<String>,   // geniş format: "c1,c2,c3"
    #[arg(long, default_value = "0.05")] pub alpha: f64,
    #[arg(long)] pub format: Option<String>,
}

#[derive(Parser)]
pub struct CiArgs {
    pub file: Option<PathBuf>,
    #[arg(long, value_parser=["mean","proportion","variance"])]
    pub ci_type: String,
    #[arg(long, default_value = "0")] pub col: String,
    #[arg(long, default_value = "0.95")] pub level: f64,
    #[arg(long)] pub successes: Option<usize>,
    #[arg(long)] pub trials: Option<usize>,
    #[arg(long, default_value="wilson", value_parser=["wilson","wald"])]
    pub method: String,
    #[arg(long)] pub format: Option<String>,
}
```

#### `io.rs`'e eklenecek fonksiyonlar

```rust
/// Uzun formattan grupları okur: her unique değer ayrı grup
pub fn read_long_groups(
    file: &Option<PathBuf>,
    value_col: &str,
    group_col: &str,
) -> Result<Vec<(String, Vec<f64>)>>
// Implement: CSV okurken HashMap<String, Vec<f64>> doldur, sonra sort_by_key

/// Geniş formattan grupları okur: "c1,c2,c3" → Vec<Vec<f64>>
pub fn read_wide_groups(
    file: &Option<PathBuf>,
    cols_spec: &str,
) -> Result<Vec<Vec<f64>>>
// Implement: cols_spec.split(',').map(|c| read_column(file, c)).collect()
```

#### ANOVA tablo çıktısı hedefi

```
One-Way ANOVA
══════════════════════════════════════════════════════
 Kaynak     SS         df    MS         F       p
──────────────────────────────────────────────────────
 Gruplar    72.000      2   36.000    27.000  0.0011
 Hata       12.000      9    1.333
──────────────────────────────────────────────────────
 Toplam     84.000     11
══════════════════════════════════════════════════════
 η²=0.857  ω²=0.821  |  H0: REDDEDİLDİ (α=0.05)
```

#### Refactor: `render.rs` → `render/` dizini (Faz 2 ÖNCE yapılacak)

```
rstat-cli/src/render/
  mod.rs      → OutputFormat + detect()
  ttest.rs    → print_ttest() taşındı
  summary.rs  → print_summary() taşındı
  anova.rs    → print_anova()   [Faz 2 yeni]
  ci.rs       → print_ci()     [Faz 2 yeni]
  chisq.rs    → print_chisq()  [Faz 3 yeni]
  spc.rs      → print_spc()    [Faz 4 yeni]
  capability.rs → print_capability() [Faz 5 yeni]
```

---

## 3. Faz 3: Chi-Square

### 3.1 Tahmini Süre: 3-4 iş günü

### 3.2 Modül: `rstat-core/src/tests_stat/chisq.rs` (SIFIRDAN)

#### Struct Tanımları

```rust
#[derive(Debug, Serialize)]
pub struct ChiSqResult {
    pub test: &'static str,         // "chi-square-independence" | "chi-square-gof"
    pub chi_sq: f64,
    pub df: usize,
    pub p_value: f64,
    pub effect_size: f64,           // Cramér's V (independence için)
    pub alpha: f64,
    pub reject_null: bool,
    pub observed: Vec<Vec<f64>>,
    pub expected: Vec<Vec<f64>>,
    pub low_expected_warning: bool, // any(E_ij < 5)
    pub yates_correction: bool,
}
```

#### Fonksiyon İmzaları

```rust
/// Bağımsızlık testi — kontenjans tablosu
pub fn independence(
    table: &[Vec<f64>],
    yates: bool,
    alpha: f64,
) -> Result<ChiSqResult, StatError>

/// Uyum iyiliği testi
pub fn goodness_of_fit(
    observed: &[f64],
    expected: &[f64],
    alpha: f64,
) -> Result<ChiSqResult, StatError>

// İç yardımcılar:
fn expected_frequencies(table: &[Vec<f64>]) -> Vec<Vec<f64>>
fn cramers_v(chi_sq: f64, n: f64, r: usize, c: usize) -> f64
```

#### Algoritmalar

```
E_ij = (R_i * C_j) / N

// Yates'siz:
χ² = Σᵢⱼ (O_ij - E_ij)² / E_ij

// Yates düzeltmeli (sadece 2×2):
χ² = Σᵢⱼ (|O_ij - E_ij| - 0.5)² / E_ij

df = (R-1)(C-1)
p = ChiSquared(df).sf(χ²)

// Cramér's V:
V = sqrt(χ² / (N * min(R-1, C-1)))

// Uyarı:
low_expected_warning = E_ij.iter().flatten().any(|&e| e < 5.0)
```

#### IO Katmanı: `read_contingency_matrix`

```rust
// io.rs'e eklenecek
/// CSV'yi kontenjans matrisine çevirir.
/// Long format: row_col + col_col (her satır = 1 gözlem, veya count_col ile ağırlıklı)
/// Matrix format: CSV zaten 2D tablo
pub fn read_contingency_matrix(
    file: &Option<PathBuf>,
    row_col: Option<&str>,
    col_col: Option<&str>,
    count_col: Option<&str>,
) -> Result<Vec<Vec<f64>>>
```

#### Edge Case'ler

```rust
if expected.iter().flatten().any(|&e| e == 0.0) {
    return Err(StatError::Numerical("Beklenen frekans 0".into()));
}
if r < 2 || c < 2 {
    return Err(StatError::InsufficientData { required: 4, got: r*c });
}
if yates && (r != 2 || c != 2) {
    return Err(StatError::InvalidParameter("Yates sadece 2×2 için".into()));
}
```

#### `scripts/crossvalidate_chisq.py`

```python
from scipy.stats import chi2_contingency, chisquare
import numpy as np, json

results = {}

# Bağımsızlık
obs = np.array([[10,20,30],[6,9,17]])
chi2, p, dof, ex = chi2_contingency(obs)
results["independence_3x2"] = {"chi_sq": chi2, "p_value": p, "df": dof}

# 2×2 Yates
obs2 = np.array([[10,20],[6,9]])
chi2, p, dof, ex = chi2_contingency(obs2, correction=True)
results["independence_2x2_yates"] = {"chi_sq": chi2, "p_value": p}

# Uyum iyiliği
chi2, p = chisquare([10,20,30,40], [25,25,25,25])
results["gof_uniform"] = {"chi_sq": chi2, "p_value": p}

print(json.dumps(results, indent=2))
```

---

## 4. Faz 4: SPC (X-bar & R Kontrol Grafikleri)

### 4.1 Tahmini Süre: 7-9 iş günü (en karmaşık faz)

### 4.2 Lisanslı Özellik Entegrasyon Stratejisi

**Karar: Runtime check, feature flag değil.**

```rust
// rstat-core/src/license.rs — Faz 4 başında stub, Faz 6'da gerçek impl

pub trait LicenseChecker: Send + Sync {
    fn check(&self) -> Result<(), LicenseError>;
}

// Üretim (Faz 6'da):
pub struct Ed25519Checker { pub_key: &'static [u8; 32] }

// Test stub (her zaman geçer):
pub struct AlwaysValid;
impl LicenseChecker for AlwaysValid {
    fn check(&self) -> Result<(), LicenseError> { Ok(()) }
}

// StatError'a yeni varyant (Faz 4 öncesi ekle):
#[error("lisanslı özellik: {0}")]
LicenseRequired(String),
```

Neden feature flag değil:
- Tek binary → kullanıcı deneyimi daha iyi (lisans dosyası koyunca açılır)
- İki binary → release pipeline, install script, brew formula hepsi ikiye katlanır
- Ed25519 doğrulaması < 1ms → performans etkisi ihmal edilebilir

### 4.3 Modül: `rstat-core/src/spc/xbar_r.rs` (SIFIRDAN)

#### Kontrol Limit Sabitleri

```rust
// ASTM STP-15D + Montgomery Table VI — iki kaynaktan doğrulanmış
pub const CONTROL_CONSTANTS: &[(usize, f64, f64, f64, f64)] = &[
    // (n,   A2,    D3,    D4,    d2)
    (2,  1.880, 0.000, 3.267, 1.128),
    (3,  1.023, 0.000, 2.574, 1.693),
    (4,  0.729, 0.000, 2.282, 2.059),
    (5,  0.577, 0.000, 2.114, 2.326),
    (6,  0.483, 0.000, 2.004, 2.534),
    (7,  0.419, 0.076, 1.924, 2.704),
    (8,  0.373, 0.136, 1.864, 2.847),
    (9,  0.337, 0.184, 1.816, 2.970),
    (10, 0.308, 0.223, 1.777, 3.078),
];

pub fn get_constants(n: usize) -> Result<(f64, f64, f64, f64), StatError> {
    CONTROL_CONSTANTS.iter()
        .find(|&&(size, ..)| size == n)
        .map(|&(_, a2, d3, d4, d2)| (a2, d3, d4, d2))
        .ok_or_else(|| StatError::InvalidParameter(
            format!("alt grup boyutu {n} desteklenmiyor (2-10)")
        ))
}
```

#### Struct Tanımları

```rust
#[derive(Debug, Serialize)]
pub struct SubgroupStat { pub subgroup: usize, pub mean: f64, pub range: f64 }

#[derive(Debug, Serialize)]
pub struct ControlLimits { pub ucl: f64, pub center: f64, pub lcl: f64 }

#[derive(Debug, Serialize)]
pub struct SpcResult {
    pub chart_type: &'static str,    // "xbar-r"
    pub n: usize,
    pub subgroups: Vec<SubgroupStat>,
    pub xbar_limits: ControlLimits,
    pub r_limits: ControlLimits,
    pub grand_mean: f64,
    pub mean_range: f64,
    pub sigma_estimate: f64,         // R̄ / d2
    pub violations: Vec<SpcViolation>,
}

#[derive(Debug, Serialize)]
pub struct SpcViolation {
    pub rule: &'static str,
    pub subgroup: usize,
    pub chart: &'static str,
    pub description: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SpcRules { WesternElectric, Nelson, None }
```

#### Kontrol Limiti Formülleri

```
X-bar chart:
  CL  = x̄̄  (grand mean)
  UCL = x̄̄ + A2 * R̄
  LCL = x̄̄ - A2 * R̄

R chart:
  CL  = R̄  (mean range)
  UCL = D4 * R̄
  LCL = D3 * R̄  (0 for n < 7)

σ_estimate = R̄ / d2
```

#### `rstat-core/src/spc/rules.rs`

```rust
/// Western Electric 4 kural seti
pub fn apply_western_electric(
    values: &[f64],
    ucl: f64, center: f64, lcl: f64,
    chart: &'static str,
) -> Vec<SpcViolation>
// Kural 1: 1 nokta 3σ dışında (UCL veya LCL geçildi)
// Kural 2: 9 ardışık nokta orta çizgi aynı tarafında
// Kural 3: 6 ardışık nokta monoton artış veya azalış
// Kural 4: 14 ardışık nokta alternatif yukarı/aşağı

/// Nelson kural seti (8 kural — daha kapsamlı)
pub fn apply_nelson(
    values: &[f64],
    ucl: f64, center: f64, lcl: f64, sigma: f64,
    chart: &'static str,
) -> Vec<SpcViolation>
```

#### R Cross-Validation (`scripts/crossvalidate_spc.R`)

```r
# qcc paketi — endüstri standardı referans
library(qcc); library(jsonlite)
data <- c(1.1,1.2,1.0,1.3,1.1,1.4,1.2,1.1,1.3,1.2,1.0,1.1,1.2,1.3,1.4)
m <- matrix(data, ncol=5, byrow=TRUE)
result <- qcc(m, type="xbar")
cat(toJSON(list(
  ucl=result$limits[,"UCL"][1],
  lcl=result$limits[,"LCL"][1],
  cl=result$center
), auto_unbox=TRUE, pretty=TRUE))
```

#### SPC Sabit Tablosu Doğrulama Testi

```rust
// Derleme zamanı assertion — tablodaki değerler ASTM ile uyuşmalı
#[test]
fn test_constants_n5() {
    let (a2, d3, d4, d2) = get_constants(5).unwrap();
    assert!((a2 - 0.577).abs() < 0.001);
    assert!((d3 - 0.000).abs() < 0.001);
    assert!((d4 - 2.114).abs() < 0.001);
    assert!((d2 - 2.326).abs() < 0.001);
}
```

---

## 5. Faz 5: Process Capability

### 5.1 Tahmini Süre: 3-4 iş günü

### 5.2 Modül: `rstat-core/src/capability/mod.rs` (SIFIRDAN)

**Faz 4 tamamlandıktan başlayın** — `sigma_estimate` (R̄/d2) Faz 4 çıktısından gelir.

#### Struct Tanımları

```rust
#[derive(Debug, Serialize)]
pub struct CapabilityResult {
    pub n: usize,
    pub mean: f64,
    pub lsl: Option<f64>,
    pub usl: Option<f64>,
    pub target: Option<f64>,
    // Within sigma (kısa vadeli)
    pub sigma_within: f64,    // R̄/d2 veya s̄/c4 (SPC'den) ya da genel std yaklaşımı
    // Overall sigma (uzun vadeli)
    pub sigma_overall: f64,   // std_dev(data) ile hesaplanır
    // Capability (within)
    pub cp:  Option<f64>,     // (USL-LSL)/(6*σ_w) — sadece çift limitli
    pub cpl: Option<f64>,     // (μ-LSL)/(3*σ_w)
    pub cpu: Option<f64>,     // (USL-μ)/(3*σ_w)
    pub cpk: f64,             // min(Cpl, Cpu)
    // Performance (overall)
    pub pp:  Option<f64>,
    pub ppl: Option<f64>,
    pub ppu: Option<f64>,
    pub ppk: f64,
    // Sigma level + PPM
    pub sigma_level: f64,     // 3 * Cpk (yaklaşık)
    pub ppm_within: f64,
    pub ppm_overall: f64,
    // Uyarı
    pub normality_warning: bool,  // |skewness| > 1.0
}
```

#### Fonksiyon İmzası

```rust
pub fn capability(
    data: &[f64],
    lsl: Option<f64>,
    usl: Option<f64>,
    target: Option<f64>,
    sigma_within: Option<f64>,  // SPC'den geliyorsa; None → sigma_overall kullan
    license: &dyn LicenseChecker,
) -> Result<CapabilityResult, StatError>
```

#### Algoritmalar

```rust
// σ_overall = std_dev(data)   [Bessel, n-1]
// σ_within = sigma_within.unwrap_or(σ_overall)

// Cp  = (usl - lsl) / (6.0 * σ_w)    [sadece her ikisi de Some ise]
// Cpl = (μ - lsl) / (3.0 * σ_w)       [sadece lsl Some ise]
// Cpu = (usl - μ) / (3.0 * σ_w)       [sadece usl Some ise]
// Cpk = [Cpl, Cpu].iter().filter_map(|&x| x).fold(f64::INFINITY, f64::min)

// PPK: aynı formüller σ_overall ile

// Sigma level ≈ 3 * Cpk

// PPM: Normal dağılım varsayımı
// statrs::Normal::new(μ, σ_w)?
// ppm = (norm.sf(usl_or_inf) + norm.cdf(lsl_or_neg_inf)) * 1_000_000.0

fn skewness(data: &[f64]) -> f64 {
    let m = mean(data).unwrap();
    let s = std_dev(data).unwrap();
    let n = data.len() as f64;
    data.iter().map(|&x| ((x - m) / s).powi(3)).sum::<f64>() / n
}
// normality_warning = skewness(data).abs() > 1.0
```

---

## 6. Çapraz Platform Test Stratejisi

### 6.1 Script Şablonu (Faz Bağımsız)

```python
#!/usr/bin/env python3
"""rstat cross-validation — Faz X"""
import json, sys
from scipy import stats
import numpy as np

# 1. Referans değerler üret
results = {}
# ... hesaplamalar ...
print(json.dumps(results, indent=2))
# Çıktı → tests/fixtures/phaseX_expected.json
```

### 6.2 Rust Test Pattern

```rust
// Her faz için aynı pattern:
static FIXTURE: &str = include_str!("../../../tests/fixtures/phaseX_expected.json");

#[test]
fn cross_validate_scipy() {
    let expected: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    // ... rstat hesapla, expected ile karşılaştır (1e-9 tolerans)
}
```

### 6.3 CI Gate Stratejisi

```yaml
# .github/workflows/ci.yml'e eklenecek (Faz 7)
- name: Cross-validate
  run: |
    python3 scripts/crossvalidate_anova.py > /tmp/anova_ref.json
    python3 scripts/crossvalidate_chisq.py > /tmp/chisq_ref.json
    cargo test --test cross_validation  # fixture'ları karşılaştırır
```

---

## 7. Teknik Borç ve Risk Analizi

### 7.1 Ertelenen Kararlar

| Karar | Risk | Önerilen Çözüm Zamanı |
|---|---|---|
| `render.rs` monolitik | Faz 3'te 400+ satır olur | Faz 2 öncesi refactor |
| `io.rs` içinde `run_summary` | Sorumluluk karışıklığı | Faz 2 başında |
| Infinity input kabul ediliyor | Yanlış hesap riski | Faz 2'de `is_finite()` ekle |
| `atty` deprecated | Cargo audit uyarısı | Faz 6'da `is-terminal` swap |
| `StatError` henüz `LicenseRequired` yok | Faz 4 compile olmaz | Faz 4 öncesi ekle |

### 7.2 NaN/Edge Case Güçlendirme

```rust
// io.rs read_column — Faz 2'de güncellenecek satırlar:
// MEVCUT (satır 50-54):
let v: f64 = cell.parse()?;
if v.is_nan() { bail!("NaN"); }

// DÜZELTME:
let v: f64 = cell.parse()?;
if !v.is_finite() {
    bail!("satır {i}: sonlu olmayan değer ({v}) kabul edilmiyor");
}
```

### 7.3 En Büyük Teknik Risk: F ve χ² Dağılımları

> **F ve χ² dağılımları `pvalue.rs`'te hiç kullanılmadı.**

`statrs::FisherSnedecor` ve `statrs::ChiSquared` API'leri `StudentsT` ile aynı (`ContinuousCDF` trait), ama **ilk kullanımda sürpriz çıkabilir:**

- `FisherSnedecor::new(df1, df2)` — parametre sırası kritik (df1=between, df2=within)
- Bazı kütüphanelerde `sf()` yerine `1.0 - cdf()` kullanmak gerekiyor
- Küçük F değerlerinde sayısal hassasiyet

**Önlem:** `dist/pvalue.rs`'e wrapper fonksiyonlar ekle:

```rust
/// F istatistiğinden p-değeri (daima sağ kuyruk)
pub fn p_value_f(f: f64, df1: f64, df2: f64) -> Result<f64, StatError> {
    let dist = FisherSnedecor::new(df1, df2)
        .map_err(|e| StatError::Numerical(format!("F-dağılımı: {e}")))?;
    Ok(dist.sf(f).clamp(0.0, 1.0))
}

/// Chi-square istatistiğinden p-değeri
pub fn p_value_chi2(chi_sq: f64, df: f64) -> Result<f64, StatError> {
    let dist = ChiSquared::new(df)
        .map_err(|e| StatError::Numerical(format!("χ²-dağılımı: {e}")))?;
    Ok(dist.sf(chi_sq).clamp(0.0, 1.0))
}
```

Bu iki fonksiyon yazıldıktan sonra scipy değerleriyle 1e-9 toleransla doğrula. Sadece bu geçtikten sonra ANOVA veya chi-square hesabına geç.

### 7.4 Ed25519 Lisans Sistemi — Implementasyon İpuçları (Faz 6)

```rust
// Bağımlılıklar (Faz 6 Cargo.toml):
// ed25519-dalek = "2"
// dirs = "5"

// Lisans dosyası formatı (JSON):
// { "customer": "X", "features": ["spc","capability"],
//   "expires": "2027-12-31",
//   "signature": "<base64 ed25519 of canonical JSON>" }

// Binary içine gömülü public key:
static PUB_KEY: &[u8; 32] = include_bytes!("../keys/public.key");

// Platform bağımsız lisans dizini:
fn license_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rstat").join("license.json")
}
```

---

## 8. Sprint Sıralaması

### 8.1 Bağımlılık Grafiği

```
Faz 0 ✅ → Faz 1 ✅ → Faz 2A (ANOVA core) ──┐
                   → Faz 2B (CI core)    ──┤
                                          └→ Faz 2C (CLI/IO/render)
                                              → Faz 3 (Chi-square)
                                              → Faz 4 (SPC)
                                                   → Faz 5 (Capability)
```

### 8.2 Paralel Yapılabilecekler

```
Faz 2 içinde:
  [PARALEL]  ANOVA hesaplama + CI hesaplama (farklı dosyalar, çakışma yok)
  [SONRA]    CLI/IO/render (her ikisine bağlı)

Faz 4 içinde:
  [PARALEL]  Kontrol limit sabitleri doğrulama + SPC rules implementasyonu
  [SONRA]    ASCII terminal grafik

Faz 4-5 arası:
  [PARALEL]  Faz 5 capability core + Faz 4 CLI entegrasyon testleri
```

### 8.3 Tam Sprint Tablosu

| Sprint | Görev | Süre | Çıktı |
|---|---|---|---|
| **S0** | `render/` refactor + `io.rs` temizlik + `is_finite()` + `StatError::LicenseRequired` | 0.5 gün | Teknik borç kapatıldı |
| **S1** | `p_value_f()` + `p_value_chi2()` → scipy doğrulama | 0.5 gün | F/χ² dağılım wrapper'ları hazır |
| **S2** | `AnovaResult` + `one_way()` + unit test | 1.5 gün | 15+ unit test |
| **S3** | `CiResult` + 3 CI fn + unit test | 1.5 gün | 12+ unit test |
| **S4** | Cross-validation scripts (ANOVA + CI) | 0.5 gün | Scipy/R referans JSON |
| **S5** | ANOVA + CI CLI (io/commands/render) | 1.5 gün | `rstat anova` + `rstat ci` çalışıyor |
| **S6** | CLI entegrasyon testleri Faz 2 | 0.5 gün | 20+ CLI test |
| **S7** | Chi-square core + cross-validation | 1.5 gün | core hazır |
| **S8** | Chi-square CLI + entegrasyon test | 1 gün | `rstat chisq` çalışıyor |
| **S9** | SPC: sabitler + xbar_r() + rules | 3 gün | SPC core hazır |
| **S10** | SPC: ASCII grafik + CLI + test | 2 gün | `rstat spc` çalışıyor |
| **S11** | Capability: core + CLI + test | 3 gün | `rstat capability` çalışıyor |
| **Toplam** | | **~17 iş günü** | **Faz 2-5 tamamlandı** |

### 8.4 Sıralı Yapılması Zorunlular

1. **`StatError::LicenseRequired` Faz 4'ten önce eklenmeli** — yoksa SPC compile olmaz
2. **`render/` refactor Faz 2 CLI'dan önce** — sonradan yapmak daha pahalı
3. **`p_value_f()` ve `p_value_chi2()` ANOVA/chi-sq'dan önce** — dağılım wrapper testleri olmadan ilerlenemez
4. **SPC sabitleri doğrulanmadan Capability yazılmamalı** — `sigma_estimate = R̄/d2` hatalıysa Cpk da hatalı olur

---

## 9. Faz 2 Hazır Kod Özeti

### ✅ Doğrudan Reuse (değişiklik gereksiz)

| Mevcut Kod | Nerede Kullanılacak |
|---|---|
| `data::summary::mean()` | ANOVA grand mean + grup means |
| `data::summary::variance()` | ANOVA SS_within hesabı |
| `data::summary::std_dev()` | CI standard error |
| `dist::pvalue::critical_t()` | `mean_ci()` — t kritik değeri |
| `dist::pvalue::ci_bounds()` | `mean_ci()` — tüm işi yapıyor! |
| `dist::pvalue::Alternative` | CI komutunda one/two-sided |
| `result::GroupStats` | `AnovaResult.groups` alanı |
| `io::read_column()` | `rstat ci --type mean` |
| `render::OutputFormat::detect()` | Tüm yeni komutlar |

### 🔜 Sıfırdan Yazılacak (Faz 2)

| Bileşen | Karmaşıklık |
|---|---|
| `tests_stat::anova::one_way()` | Orta (F dağılımı yeni) |
| `interval::ci::mean_ci()` | **Düşük** (ci_bounds reuse) |
| `interval::ci::proportion_ci()` | Orta (Wilson formülü) |
| `interval::ci::variance_ci()` | Orta (χ² dağılımı yeni) |
| `io::read_long_groups()` | Orta (HashMap pivot) |
| `io::read_wide_groups()` | Düşük |
| `commands/anova.rs` | Düşük (ttest.rs kopyala-uyarla) |
| `commands/ci.rs` | Düşük |
| `render/anova.rs` | Orta (ANOVA tablosu formatı) |
| `render/ci.rs` | Düşük |

---

*Bu belge implementasyon süresince güncellenecektir.*
