use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

fn cmd() -> Command {
    Command::cargo_bin("rstat-cli").unwrap()
}

fn load_expected() -> Value {
    let content = fs::read_to_string("tests/fixtures/chisq_expected.json")
        .expect("chisq_expected.json okunamadı");
    serde_json::from_str(&content).expect("JSON ayrıştırılamadı")
}

#[test]
fn test_chisq_independence_2x2_yates() {
    let expected = load_expected();
    let exp_data = &expected["independence_2x2_yates"];

    let out = cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_2x2.csv",
            "--kind",
            "independence",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "chi-square-independence");
    assert_eq!(v["yates_corrected"], true);

    let stat = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();
    let df = v["df"].as_f64().unwrap();
    let v_coeff = v["cramers_v"].as_f64().unwrap();

    assert!((stat - exp_data["statistic"].as_f64().unwrap()).abs() < 1e-9);
    assert!((p - exp_data["p_value"].as_f64().unwrap()).abs() < 1e-9);
    assert_eq!(df, exp_data["df"].as_f64().unwrap());
    assert!((v_coeff - exp_data["cramers_v"].as_f64().unwrap()).abs() < 1e-9);

    // Beklenen frekans matrisini doğrula
    let expected_matrix = v["expected"].as_array().unwrap();
    let exp_matrix_scipy = exp_data["expected"].as_array().unwrap();
    for (i, row) in expected_matrix.iter().enumerate() {
        for (j, val) in row.as_array().unwrap().iter().enumerate() {
            let val_f = val.as_f64().unwrap();
            let scipy_f = exp_matrix_scipy[i][j].as_f64().unwrap();
            assert!((val_f - scipy_f).abs() < 1e-9);
        }
    }
}

#[test]
fn test_chisq_independence_2x2_no_yates() {
    let expected = load_expected();
    let exp_data = &expected["independence_2x2_no_yates"];

    let out = cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_2x2.csv",
            "--kind",
            "independence",
            "--no-yates",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "chi-square-independence");
    assert_eq!(v["yates_corrected"], false);

    let stat = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();

    assert!((stat - exp_data["statistic"].as_f64().unwrap()).abs() < 1e-9);
    assert!((p - exp_data["p_value"].as_f64().unwrap()).abs() < 1e-9);
}

#[test]
fn test_chisq_independence_3x3() {
    let expected = load_expected();
    let exp_data = &expected["independence_3x3"];

    let out = cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_3x3.csv",
            "--kind",
            "independence",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "chi-square-independence");
    assert_eq!(v["yates_corrected"], false); // 3x3 olduğu için yates otomatik olarak false olmalı

    let stat = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();
    let df = v["df"].as_f64().unwrap();
    let v_coeff = v["cramers_v"].as_f64().unwrap();

    assert!((stat - exp_data["statistic"].as_f64().unwrap()).abs() < 1e-9);
    assert!((p - exp_data["p_value"].as_f64().unwrap()).abs() < 1e-9);
    assert_eq!(df, exp_data["df"].as_f64().unwrap());
    assert!((v_coeff - exp_data["cramers_v"].as_f64().unwrap()).abs() < 1e-9);
}

#[test]
fn test_chisq_independence_long_yates() {
    let expected = load_expected();
    let exp_data = &expected["independence_long_yates"];

    let out = cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_long.csv",
            "--kind",
            "independence",
            "--col1",
            "var1",
            "--col2",
            "var2",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "chi-square-independence");
    assert_eq!(v["yates_corrected"], true);

    let stat = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();

    assert!((stat - exp_data["statistic"].as_f64().unwrap()).abs() < 1e-9);
    assert!((p - exp_data["p_value"].as_f64().unwrap()).abs() < 1e-9);
}

#[test]
fn test_chisq_gof() {
    let expected = load_expected();
    let exp_data = &expected["gof"];

    let out = cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_gof.csv",
            "--kind",
            "gof",
            "--col1",
            "observed",
            "--col2",
            "expected_prop",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "chi-square-goodness-of-fit");

    let stat = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();
    let df = v["df"].as_f64().unwrap();

    assert!((stat - exp_data["statistic"].as_f64().unwrap()).abs() < 1e-9);
    assert!((p - exp_data["p_value"].as_f64().unwrap()).abs() < 1e-9);
    assert_eq!(df, exp_data["df"].as_f64().unwrap());
}

#[test]
fn test_chisq_table_format() {
    cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_2x2.csv",
            "--kind",
            "independence",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ki-Kare Bağımsızlık Testi"))
        .stdout(predicate::str::contains("Gözlenen (Observed)"))
        .stdout(predicate::str::contains("Ki-Kare İstatistiği (χ²)"))
        .stdout(predicate::str::contains("Yates Düzeltmesi Uygulandı"));
}

#[test]
fn test_chisq_csv_format() {
    cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_2x2.csv",
            "--kind", "independence",
            "--format", "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test,statistic,df,p_value,cramers_v,yates_corrected,warning_low_expected,alpha,reject_null"))
        .stdout(predicate::str::contains("chi-square-independence,0.1280000000,1,0.7205147871,0.0666666667,true,false,0.0500,false"));
}

#[test]
fn test_chisq_missing_file_fails() {
    cmd()
        .args(["chisq", "nonexistent_file.csv"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dosya açılamadı"));
}

#[test]
fn test_chisq_invalid_alpha_fails() {
    cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_2x2.csv",
            "--alpha",
            "2.0",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("alpha (0, 1) aralığında olmalı"));
}

#[test]
fn test_chisq_gof_missing_cols_fails() {
    cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_gof.csv",
            "--kind",
            "gof",
            "--col1",
            "observed",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Uyum iyiliği testi için beklenen kolon",
        ));
}

#[test]
fn test_chisq_independence_partial_cols_fails() {
    cmd()
        .args([
            "chisq",
            "tests/fixtures/chisq_independence_long.csv",
            "--kind",
            "independence",
            "--col1",
            "var1",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Bağımsızlık testi uzun formatı için hem --col1 hem de --col2 belirtilmeli",
        ));
}
