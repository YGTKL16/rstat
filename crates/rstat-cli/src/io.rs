use anyhow::{Context, Result, bail};
use rstat_core::data::summary::summary;
use crate::cli::SummaryArgs;
use crate::render::{OutputFormat, print_summary};
use std::io::Read;
use std::path::PathBuf;

pub fn run_summary(args: SummaryArgs) -> Result<()> {
    let data = read_column(&args.file, &args.col)?;
    let stats = summary(&data).context("özet istatistik hesaplanamadı")?;
    let fmt = OutputFormat::detect(args.format.as_deref());
    print_summary(&stats, fmt);
    Ok(())
}

pub(crate) fn read_column(file: &Option<std::path::PathBuf>, col: &str) -> Result<Vec<f64>> {
    let reader: Box<dyn Read> = match file {
        Some(path) => Box::new(
            std::fs::File::open(path)
                .with_context(|| format!("dosya açılamadı: {}", path.display()))?,
        ),
        None => Box::new(std::io::stdin()),
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    // Kolon indeksini belirle (isim veya sayı)
    let col_idx = if let Ok(n) = col.parse::<usize>() {
        n
    } else {
        let headers = rdr.headers()?.clone();
        headers
            .iter()
            .position(|h| h == col)
            .with_context(|| format!("kolon bulunamadı: '{col}'"))?
    };

    let mut values: Vec<f64> = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;
        let cell = record.get(col_idx)
            .with_context(|| format!("satır {i}'de kolon {col_idx} yok"))?
            .trim();
        if cell.is_empty() || cell.eq_ignore_ascii_case("na") || cell.eq_ignore_ascii_case("nan") {
            continue; // NA değerleri atla
        }
        let v: f64 = cell.parse()
            .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
        if v.is_nan() {
            bail!("satır {i}: NaN değer");
        }
        values.push(v);
    }

    if values.is_empty() {
        bail!("kolon boş veya tüm değerler NA");
    }

    Ok(values)
}

/// İki kolonu aynı dosyadan okur. Paired için satır eşleştirmesi korunur
/// (bir satırda herhangi bir değer NA ise çift birlikte atlanır).
pub(crate) fn read_two_columns(
    file: &Option<PathBuf>,
    col1: &str,
    col2: &str,
) -> Result<(Vec<f64>, Vec<f64>)> {
    let reader: Box<dyn Read> = match file {
        Some(path) => Box::new(
            std::fs::File::open(path)
                .with_context(|| format!("dosya açılamadı: {}", path.display()))?,
        ),
        None => Box::new(std::io::stdin()),
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr.headers()?.clone();
    let resolve_col = |col: &str| -> Result<usize> {
        if let Ok(n) = col.parse::<usize>() {
            Ok(n)
        } else {
            headers
                .iter()
                .position(|h| h == col)
                .with_context(|| format!("kolon bulunamadı: '{col}'"))
        }
    };
    let idx1 = resolve_col(col1)?;
    let idx2 = resolve_col(col2)?;

    let mut a: Vec<f64> = Vec::new();
    let mut b: Vec<f64> = Vec::new();

    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;
        let parse_cell = |idx: usize| -> Result<Option<f64>> {
            let cell = record.get(idx)
                .with_context(|| format!("satır {i}'de kolon {idx} yok"))?
                .trim();
            if cell.is_empty() || cell.eq_ignore_ascii_case("na") || cell.eq_ignore_ascii_case("nan") {
                return Ok(None);
            }
            let v: f64 = cell.parse()
                .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
            Ok(Some(v))
        };

        match (parse_cell(idx1)?, parse_cell(idx2)?) {
            (Some(v1), Some(v2)) => { a.push(v1); b.push(v2); }
            _ => {} // bir taraf NA → çifti birlikte atla
        }
    }

    if a.is_empty() {
        bail!("kolonlar boş veya tüm satırlar NA");
    }

    Ok((a, b))
}
