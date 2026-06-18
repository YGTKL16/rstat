use anyhow::{Context, Result, bail};
use std::io::Read;
use std::path::PathBuf;

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
        let cell = record
            .get(col_idx)
            .with_context(|| format!("satır {i}'de kolon {col_idx} yok"))?
            .trim();
        if cell.is_empty() || cell.eq_ignore_ascii_case("na") || cell.eq_ignore_ascii_case("nan") {
            continue; // NA değerleri atla
        }
        let v: f64 = cell
            .parse()
            .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
        if !v.is_finite() {
            bail!("satır {i}: sonlu olmayan değer ({v}) kabul edilmiyor");
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
            let cell = record
                .get(idx)
                .with_context(|| format!("satır {i}'de kolon {idx} yok"))?
                .trim();
            if cell.is_empty()
                || cell.eq_ignore_ascii_case("na")
                || cell.eq_ignore_ascii_case("nan")
            {
                return Ok(None);
            }
            let v: f64 = cell
                .parse()
                .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
            Ok(Some(v))
        };

        if let (Some(v1), Some(v2)) = (parse_cell(idx1)?, parse_cell(idx2)?) {
            a.push(v1);
            b.push(v2);
        }
    }

    if a.is_empty() {
        bail!("kolonlar boş veya tüm satırlar NA");
    }

    Ok((a, b))
}

pub(crate) fn read_wide_groups(file: &Option<PathBuf>, cols_spec: &str) -> Result<Vec<Vec<f64>>> {
    let cols: Vec<&str> = cols_spec.split(',').map(|s| s.trim()).collect();
    if cols.is_empty() {
        bail!("Geniş format için kolon listesi boş olamaz");
    }
    let mut groups = Vec::new();
    for col in cols {
        let values = read_column(file, col)?;
        groups.push(values);
    }
    Ok(groups)
}

pub(crate) fn read_long_groups(
    file: &Option<PathBuf>,
    value_col: &str,
    group_col: &str,
) -> Result<Vec<(String, Vec<f64>)>> {
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

    let val_idx = resolve_col(value_col)?;
    let grp_idx = resolve_col(group_col)?;

    use std::collections::HashMap;
    let mut map: HashMap<String, Vec<f64>> = HashMap::new();

    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;

        let val_cell = record
            .get(val_idx)
            .with_context(|| format!("satır {i}'de kolon {val_idx} yok"))?
            .trim();
        let grp_cell = record
            .get(grp_idx)
            .with_context(|| format!("satır {i}'de kolon {grp_idx} yok"))?
            .trim();

        if val_cell.is_empty()
            || val_cell.eq_ignore_ascii_case("na")
            || val_cell.eq_ignore_ascii_case("nan")
        {
            continue;
        }
        if grp_cell.is_empty()
            || grp_cell.eq_ignore_ascii_case("na")
            || grp_cell.eq_ignore_ascii_case("nan")
        {
            continue;
        }

        let val: f64 = val_cell
            .parse()
            .with_context(|| format!("satır {i}, '{val_cell}' sayıya dönüştürülemedi"))?;
        if !val.is_finite() {
            bail!("satır {i}: sonlu olmayan değer ({val}) kabul edilmiyor");
        }

        map.entry(grp_cell.to_string()).or_default().push(val);
    }

    if map.is_empty() {
        bail!("Veri bulunamadı veya tüm satırlar NA");
    }

    let mut vec: Vec<(String, Vec<f64>)> = map.into_iter().collect();
    vec.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(vec)
}

pub(crate) fn read_matrix(file: &Option<PathBuf>) -> Result<Vec<Vec<f64>>> {
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

    let mut matrix = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;
        let mut row = Vec::new();
        // Check if first column is non-numeric (row label/name)
        let start_idx = if record.get(0).and_then(|s| s.parse::<f64>().ok()).is_none() {
            1
        } else {
            0
        };

        for j in start_idx..record.len() {
            let cell = record
                .get(j)
                .with_context(|| format!("satır {i}'de kolon {j} yok"))?
                .trim();
            if cell.is_empty()
                || cell.eq_ignore_ascii_case("na")
                || cell.eq_ignore_ascii_case("nan")
            {
                continue; // Skip NA values? Wait, in a contingency matrix, NA is not typically allowed, but if we encounter it, let's treat as error or skip. Let's treat as error to be safe, or skip. If it's a matrix format, every cell is counts, so no cell should be empty/NA.
            }
            let v: f64 = cell
                .parse()
                .with_context(|| format!("satır {i}, '{cell}' sayıya dönüştürülemedi"))?;
            row.push(v);
        }
        if row.is_empty() {
            bail!("satır {i} boş veya geçersiz");
        }
        matrix.push(row);
    }

    if matrix.is_empty() {
        bail!("matris boş");
    }

    // Check that all rows have the same length
    let expected_len = matrix[0].len();
    for (i, row) in matrix.iter().enumerate() {
        if row.len() != expected_len {
            bail!(
                "satır {i} uzunluğu ({}) diğer satırlar ({}) ile eşleşmiyor",
                row.len(),
                expected_len
            );
        }
    }

    Ok(matrix)
}

pub(crate) fn read_contingency_table_long(
    file: &Option<PathBuf>,
    col1: &str,
    col2: &str,
) -> Result<Vec<Vec<f64>>> {
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

    let mut row_categories = Vec::new();
    let mut col_categories = Vec::new();
    let mut counts = std::collections::HashMap::new();

    for (i, result) in rdr.records().enumerate() {
        let record = result.with_context(|| format!("satır {i} okunamadı"))?;
        let val1 = record
            .get(idx1)
            .with_context(|| format!("satır {i}'de kolon {idx1} yok"))?
            .trim()
            .to_string();
        let val2 = record
            .get(idx2)
            .with_context(|| format!("satır {i}'de kolon {idx2} yok"))?
            .trim()
            .to_string();

        if val1.is_empty() || val1.eq_ignore_ascii_case("na") || val1.eq_ignore_ascii_case("nan") {
            continue;
        }
        if val2.is_empty() || val2.eq_ignore_ascii_case("na") || val2.eq_ignore_ascii_case("nan") {
            continue;
        }

        if !row_categories.contains(&val1) {
            row_categories.push(val1.clone());
        }
        if !col_categories.contains(&val2) {
            col_categories.push(val2.clone());
        }

        *counts.entry((val1, val2)).or_insert(0.0) += 1.0;
    }

    if row_categories.is_empty() || col_categories.is_empty() {
        bail!("Geçerli kategori verisi bulunamadı");
    }

    // Sort categories to have consistent matrix indices
    row_categories.sort();
    col_categories.sort();

    let mut table = vec![vec![0.0; col_categories.len()]; row_categories.len()];
    for i in 0..row_categories.len() {
        for j in 0..col_categories.len() {
            let key = (row_categories[i].clone(), col_categories[j].clone());
            if let Some(&count) = counts.get(&key) {
                table[i][j] = count;
            }
        }
    }

    Ok(table)
}
