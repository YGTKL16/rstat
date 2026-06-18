use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rstat-cli").unwrap()
}

#[test]
fn test_anova_wide_json() {
    let out = cmd()
        .args([
            "anova",
            "tests/fixtures/anova_three_groups.csv",
            "--cols",
            "group1,group2,group3",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "one-way-anova");
    assert_eq!(v["ss_between"], 10.0);
    assert_eq!(v["ss_within"], 30.0);
    assert_eq!(v["ss_total"], 40.0);
    assert_eq!(v["df_between"], 2.0);
    assert_eq!(v["df_within"], 12.0);
    assert_eq!(v["f_statistic"], 2.0);
    assert!((v["p_value"].as_f64().unwrap() - 0.17797851562500003).abs() < 1e-9);
    assert_eq!(v["eta_squared"], 0.25);
    assert_eq!(v["reject_null"], false);
}

#[test]
fn test_anova_wide_csv() {
    cmd()
        .args([
            "anova",
            "tests/fixtures/anova_three_groups.csv",
            "--cols",
            "group1,group2,group3",
            "--format",
            "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "source,ss,df,ms,f,p_value,eta_squared,reject_null",
        ))
        .stdout(predicate::str::contains(
            "between,10.0000000000,2,5.0000000000,2.0000000000,0.1779785156",
        ))
        .stdout(predicate::str::contains(
            "within,30.0000000000,12,2.5000000000",
        ))
        .stdout(predicate::str::contains("total,40.0000000000,14"));
}

#[test]
fn test_anova_wide_table() {
    cmd()
        .args([
            "anova",
            "tests/fixtures/anova_three_groups.csv",
            "--cols",
            "group1,group2,group3",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("One-Way ANOVA"))
        .stdout(predicate::str::contains("Gruplar"))
        .stdout(predicate::str::contains("Hata"))
        .stdout(predicate::str::contains("Toplam"))
        .stdout(predicate::str::contains("η²=0.250"))
        .stdout(predicate::str::contains("H0: REDDEDİLEMEDİ"));
}

#[test]
fn test_anova_long_json() {
    let out = cmd()
        .args([
            "anova",
            "tests/fixtures/anova_long.csv",
            "--value",
            "value",
            "--group",
            "group",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "one-way-anova");
    assert_eq!(v["ss_between"], 10.0);
    assert_eq!(v["ss_within"], 30.0);
    assert_eq!(v["f_statistic"], 2.0);
    assert!((v["p_value"].as_f64().unwrap() - 0.17797851562500003).abs() < 1e-9);
}

#[test]
fn test_anova_long_csv() {
    cmd()
        .args([
            "anova",
            "tests/fixtures/anova_long.csv",
            "--value",
            "value",
            "--group",
            "group",
            "--format",
            "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "between,10.0000000000,2,5.0000000000,2.0000000000,0.1779785156",
        ));
}

#[test]
fn test_anova_long_table() {
    cmd()
        .args([
            "anova",
            "tests/fixtures/anova_long.csv",
            "--value",
            "value",
            "--group",
            "group",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("One-Way ANOVA"))
        .stdout(predicate::str::contains("H0: REDDEDİLEMEDİ"));
}

#[test]
fn test_anova_missing_cols_fails() {
    cmd()
        .args(["anova", "tests/fixtures/anova_three_groups.csv"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ya --cols"));
}

#[test]
fn test_anova_invalid_alpha_fails() {
    cmd()
        .args([
            "anova",
            "tests/fixtures/anova_three_groups.csv",
            "--cols",
            "group1,group2,group3",
            "--alpha",
            "1.5",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("alpha (0, 1) aralığında olmalı"));
}

#[test]
fn test_anova_nonexistent_file_fails() {
    cmd()
        .args(["anova", "nonexistent.csv", "--cols", "group1,group2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dosya açılamadı"));
}
