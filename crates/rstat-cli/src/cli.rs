use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rstat", version, about = "Pipeline-friendly istatistik CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Özet istatistikler (n, mean, std, min, Q1, median, Q3, max)
    Summary(SummaryArgs),
    /// t-testi (one-sample / two-sample / paired)
    Ttest(TtestArgs),
    /// ANOVA (Tek Yönlü Varyans Analizi)
    Anova(AnovaArgs),
    /// Güven Aralıkları (Mean, Proportion, Variance)
    Ci(CiArgs),
    /// Ki-Kare Testleri (Bağımsızlık ve Uyum İyiliği)
    Chisq(ChisqArgs),
    /// İstatistiki Süreç Kontrolü (SPC) X-bar & R Grafiği
    Spc(SpcArgs),
    /// Proses Yeterlilik Analizi (Cp, Cpk, Pp, Ppk)
    Capability(CapabilityArgs),
}

#[derive(Parser)]
pub struct AnovaArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Gözlem değeri kolonu (uzun format)
    #[arg(long)]
    pub value: Option<String>,

    /// Grup etiket kolonu (uzun format)
    #[arg(long)]
    pub group: Option<String>,

    /// Kolon isimleri (geniş format, virgülle ayrılmış: "g1,g2,g3")
    #[arg(long)]
    pub cols: Option<String>,

    /// Anlamlılık düzeyi
    #[arg(long, default_value = "0.05")]
    pub alpha: f64,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct CiArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Güven aralığı türü: mean, proportion, variance
    #[arg(long, value_parser = ["mean", "proportion", "variance"])]
    pub ci_type: String,

    /// Kolon adı veya indeksi (mean ve variance için)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// Güven düzeyi
    #[arg(long, default_value = "0.95")]
    pub level: f64,

    /// Başarı sayısı (proportion için alternatif girdi)
    #[arg(long)]
    pub successes: Option<u64>,

    /// Deneme sayısı (proportion için alternatif girdi)
    #[arg(long)]
    pub trials: Option<u64>,

    /// Oran güven aralığı yöntemi: wilson (varsayılan) | wald
    #[arg(long, default_value = "wilson", value_parser = ["wilson", "wald"])]
    pub method: String,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct TtestArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Test türü: one, two, paired
    #[arg(long, value_parser = ["one", "two", "paired"])]
    pub kind: String,

    /// Birinci kolon (ad veya 0-tabanlı indeks)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// İkinci kolon (two/paired için zorunlu)
    #[arg(long)]
    pub col2: Option<String>,

    /// one-sample için H0 ortalaması
    #[arg(long, default_value = "0.0")]
    pub mu: f64,

    /// Varyans varsayımı (yalnızca two): welch (varsayılan) | pooled
    #[arg(long, default_value = "welch", value_parser = ["welch", "pooled"])]
    pub var: String,

    /// Alternatif hipotez
    #[arg(long, default_value = "two-sided", value_parser = ["two-sided", "less", "greater"])]
    pub alt: String,

    /// Güven düzeyi (CI için)
    #[arg(long, default_value = "0.95")]
    pub ci_level: f64,

    /// Anlamlılık düzeyi
    #[arg(long, default_value = "0.05")]
    pub alpha: f64,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct SummaryArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Kullanılacak kolon adı veya indeksi (0-tabanlı)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct ChisqArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Test türü: independence (varsayılan) | gof
    #[arg(long, value_parser = ["independence", "gof"], default_value = "independence")]
    pub kind: String,

    /// Bağımsızlık testi için 1. kolon (long format) veya gof için gözlenen kolon
    #[arg(long)]
    pub col1: Option<String>,

    /// Bağımsızlık testi için 2. kolon (long format) veya gof için beklenen kolon
    #[arg(long)]
    pub col2: Option<String>,

    /// Yates düzeltmesini kapatır (yalnızca 2x2 bağımsızlık testi için geçerli)
    #[arg(long)]
    pub no_yates: bool,

    /// Anlamlılık düzeyi
    #[arg(long, default_value = "0.05")]
    pub alpha: f64,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct SpcArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Kullanılacak kolon adı veya indeksi (0-tabanlı)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// Alt grup boyutu (2-10 arası)
    #[arg(long, default_value = "5")]
    pub subgroup_size: usize,

    /// Grafik türü: xbar-r
    #[arg(long, default_value = "xbar-r", value_parser = ["xbar-r"])]
    pub chart: String,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Parser)]
pub struct CapabilityArgs {
    /// Girdi CSV dosyası (verilmezse stdin)
    pub file: Option<std::path::PathBuf>,

    /// Kullanılacak kolon adı veya indeksi (0-tabanlı)
    #[arg(long, default_value = "0")]
    pub col: String,

    /// Alt grup boyutu (2-10 arası)
    #[arg(long, default_value = "5")]
    pub subgroup_size: usize,

    /// Alt spesifikasyon limiti (LSL)
    #[arg(long)]
    pub lsl: Option<f64>,

    /// Üst spesifikasyon limiti (USL)
    #[arg(long)]
    pub usl: Option<f64>,

    /// Hedef değer (Target)
    #[arg(long)]
    pub target: Option<f64>,

    /// Çıktı formatı: table, json, csv
    #[arg(long)]
    pub format: Option<String>,
}
