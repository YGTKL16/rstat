use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rstat-cli").unwrap()
}

#[test]
fn test_ci_mean_json() {
    let out = cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "mean",
            "--col",
            "value",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["n"], 6);
    assert!((v["mean"].as_f64().unwrap() - 2.75).abs() < 1e-9);
    assert!((v["ci"][0].as_f64().unwrap() - 1.7683428465098379).abs() < 1e-9);
    assert!((v["ci"][1].as_f64().unwrap() - 3.731657153490162).abs() < 1e-9);
}

#[test]
fn test_ci_mean_csv() {
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type", "mean",
            "--col", "value",
            "--format", "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("n,mean,std,se,df,ci_level,ci_low,ci_high"))
        .stdout(predicate::str::contains("6,2.7500000000,0.9354143467,0.3818813079,5.0000000000,0.9500000000,1.7683428465,3.7316571535"));
}

#[test]
fn test_ci_mean_table() {
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "mean",
            "--col",
            "value",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ortalama Güven Aralığı"))
        .stdout(predicate::str::contains("n"))
        .stdout(predicate::str::contains("mean"))
        .stdout(predicate::str::contains("std"))
        .stdout(predicate::str::contains("se"))
        .stdout(predicate::str::contains("df"))
        .stdout(predicate::str::contains("[1.768343, 3.731657]"));
}

#[test]
fn test_ci_proportion_wald_json() {
    let out = cmd()
        .args([
            "ci",
            "--ci-type",
            "proportion",
            "--successes",
            "4",
            "--trials",
            "10",
            "--method",
            "wald",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["successes"], 4);
    assert_eq!(v["trials"], 10);
    assert_eq!(v["p_hat"], 0.4);
    assert_eq!(v["method"], "wald");
    assert!((v["ci"][0].as_f64().unwrap() - 0.0963636851484016).abs() < 1e-9);
    assert!((v["ci"][1].as_f64().unwrap() - 0.7036363148515985).abs() < 1e-9);
}

#[test]
fn test_ci_proportion_wilson_json() {
    let out = cmd()
        .args([
            "ci",
            "--ci-type",
            "proportion",
            "--successes",
            "4",
            "--trials",
            "10",
            "--method",
            "wilson",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["method"], "wilson");
    assert!((v["ci"][0].as_f64().unwrap() - 0.16818032970623614).abs() < 1e-9);
    assert!((v["ci"][1].as_f64().unwrap() - 0.6873262302663417).abs() < 1e-9);
}

#[test]
fn test_ci_proportion_csv_json() {
    let out = cmd()
        .args([
            "ci",
            "tests/fixtures/ci_proportion.csv",
            "--ci-type",
            "proportion",
            "--col",
            "success",
            "--method",
            "wilson",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["successes"], 4);
    assert_eq!(v["trials"], 10);
    assert!((v["ci"][0].as_f64().unwrap() - 0.16818032970623614).abs() < 1e-9);
}

#[test]
fn test_ci_variance_json() {
    let out = cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "variance",
            "--col",
            "value",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["n"], 6);
    assert!((v["variance"].as_f64().unwrap() - 0.875).abs() < 1e-9);
    assert!((v["ci_variance"][0].as_f64().unwrap() - 0.3409311763236312).abs() < 1e-9);
    assert!((v["ci_variance"][1].as_f64().unwrap() - 5.2634009547199385).abs() < 1e-9);
    assert!((v["ci_std_dev"][0].as_f64().unwrap() - 0.58389312063393).abs() < 1e-9);
    assert!((v["ci_std_dev"][1].as_f64().unwrap() - 2.2942103117892097).abs() < 1e-9);
}

#[test]
fn test_ci_variance_csv() {
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type", "variance",
            "--col", "value",
            "--format", "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("n,variance,std_dev,df,ci_level,ci_var_low,ci_var_high,ci_std_low,ci_std_high"))
        .stdout(predicate::str::contains("6,0.8750000000,0.9354143467,5.0000000000,0.9500000000,0.3409311763,5.2634009547,0.5838931206,2.2942103118"));
}

#[test]
fn test_ci_variance_table() {
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "variance",
            "--col",
            "value",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Varyans & Std Sapma Güven Aralığı",
        ))
        .stdout(predicate::str::contains("varyans (s²)"))
        .stdout(predicate::str::contains("[0.340931, 5.263401]"))
        .stdout(predicate::str::contains("[0.583893, 2.294210]"));
}

#[test]
fn test_ci_invalid_level_fails() {
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "mean",
            "--col",
            "value",
            "--level",
            "1.5",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("level (0, 1) aralığında olmalı"));
}

#[test]
fn test_ci_invalid_type_fails() {
    // Rust CLI parser should reject invalid type directly
    cmd()
        .args([
            "ci",
            "tests/fixtures/ci_mean.csv",
            "--ci-type",
            "invalid_type",
        ])
        .assert()
        .failure();
}
