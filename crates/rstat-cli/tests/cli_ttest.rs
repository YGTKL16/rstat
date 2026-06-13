use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rstat-cli").unwrap()
}

// ── one-sample ────────────────────────────────────────────────────────────────

#[test]
fn ttest_one_sample_json() {
    let out = cmd()
        .args(["ttest", "--kind", "one", "--col", "value", "--mu", "5.0", "--format", "json"])
        .write_stdin("value\n5.1\n4.9\n6.2\n5.5\n5.0\n4.8\n6.1\n5.3\n5.7\n4.95\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["test"], "one-sample");
    assert_eq!(v["alternative"], "two-sided");

    // scipy: statistic=2.2292973506512723, p=0.05275727446494313
    let t = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();
    assert!((t - 2.2292973506512723).abs() < 1e-9, "t={t}");
    assert!((p - 0.05275727446494313).abs() < 1e-9, "p={p}");
    assert_eq!(v["reject_null"], false);
}

#[test]
fn ttest_one_sample_file() {
    let out = cmd()
        .args([
            "ttest",
            "tests/fixtures/one_col.csv",
            "--kind", "one",
            "--col", "value",
            "--mu", "5.0",
            "--format", "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let p = v["p_value"].as_f64().unwrap();
    assert!((p - 0.05275727446494313).abs() < 1e-9, "p={p}");
}

#[test]
fn ttest_one_sample_greater() {
    let out = cmd()
        .args([
            "ttest", "--kind", "one", "--col", "value",
            "--mu", "5.0", "--alt", "greater", "--format", "json",
        ])
        .write_stdin("value\n5.1\n4.9\n6.2\n5.5\n5.0\n4.8\n6.1\n5.3\n5.7\n4.95\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // scipy: p=0.026378637232471566
    let p = v["p_value"].as_f64().unwrap();
    assert!((p - 0.026378637232471566).abs() < 1e-9, "p={p}");
    // Greater → CI üst sınır sonsuz (JSON'da null olarak serialize edilir)
    assert!(v["ci"][1].is_null(), "ci[1] null (sonsuz) olmalı: {:?}", v["ci"][1]);
}

#[test]
fn ttest_one_sample_less() {
    let out = cmd()
        .args([
            "ttest", "--kind", "one", "--col", "value",
            "--mu", "5.0", "--alt", "less", "--format", "json",
        ])
        .write_stdin("value\n5.1\n4.9\n6.2\n5.5\n5.0\n4.8\n6.1\n5.3\n5.7\n4.95\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // Less → CI alt sınır sonsuz (JSON'da null olarak serialize edilir)
    assert!(v["ci"][0].is_null(), "ci[0] null (−sonsuz) olmalı: {:?}", v["ci"][0]);
}

#[test]
fn ttest_one_sample_table_output() {
    cmd()
        .args(["ttest", "--kind", "one", "--col", "value", "--mu", "5.0", "--format", "table"])
        .write_stdin("value\n5.1\n4.9\n6.2\n5.5\n5.0\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("p-değeri"))
        .stdout(predicate::str::contains("Cohen's d"));
}

// ── two-sample ────────────────────────────────────────────────────────────────

#[test]
fn ttest_two_sample_welch_json() {
    let out = cmd()
        .args([
            "ttest", "--kind", "two",
            "--col", "0", "--col2", "1",
            "--format", "json",
        ])
        .write_stdin("a,b\n2,1\n4,2\n4,3\n4,3.5\n5,4\n5,4\n7,5\n9,8\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["method"], "welch");
    // scipy: ttest_ind(A,B, equal_var=False) → p=0.28162...
    let p = v["p_value"].as_f64().unwrap();
    assert!((p - 0.28162172376376).abs() < 1e-9, "p={p}");
}

#[test]
fn ttest_two_sample_pooled() {
    let out = cmd()
        .args([
            "ttest", "--kind", "two",
            "--col", "a", "--col2", "b",
            "--var", "pooled",
            "--format", "json",
        ])
        .write_stdin("a,b\n2,1\n4,2\n4,3\n4,3.5\n5,4\n5,4\n7,5\n9,8\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["method"], "pooled");
    assert!((v["df"].as_f64().unwrap() - 14.0).abs() < 1e-9);
}

// ── paired ────────────────────────────────────────────────────────────────────

#[test]
fn ttest_paired_json() {
    let out = cmd()
        .args([
            "ttest",
            "tests/fixtures/two_col.csv",
            "--kind", "paired",
            "--col", "before",
            "--col2", "after",
            "--format", "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // scipy: ttest_rel(P1,P2) → t=7.9056..., p=0.000521...
    let t = v["statistic"].as_f64().unwrap();
    let p = v["p_value"].as_f64().unwrap();
    assert!((t - 7.905694150420959).abs() < 1e-9, "t={t}");
    assert!((p - 0.0005210669895035266).abs() < 1e-9, "p={p}");
    assert_eq!(v["reject_null"], true);
}

#[test]
fn ttest_paired_rejects_null() {
    // Güçlü etki: H0 reddedilmeli
    cmd()
        .args([
            "ttest", "--kind", "paired",
            "--col", "before", "--col2", "after",
            "--format", "json",
        ])
        .write_stdin("before,after\n20,10\n18,9\n22,11\n19,10\n21,10\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"reject_null\": true"));
}

// ── JSON şema bütünlüğü ───────────────────────────────────────────────────────

#[test]
fn ttest_json_schema_complete() {
    let out = cmd()
        .args(["ttest", "--kind", "one", "--col", "v", "--mu", "0", "--format", "json"])
        .write_stdin("v\n1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // Tüm alanlar mevcut olmalı
    for field in &["test","method","alternative","statistic","df","p_value","ci","ci_level","cohens_d","alpha","reject_null","groups"] {
        assert!(v.get(field).is_some(), "eksik alan: {field}");
    }
    // p_value [0,1] aralığında
    let p = v["p_value"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&p), "p değeri [0,1] dışında: {p}");
    // two-sided CI: her iki sınır sonlu ve alt ≤ üst
    let ci0 = v["ci"][0].as_f64().expect("ci[0] sonlu olmalı");
    let ci1 = v["ci"][1].as_f64().expect("ci[1] sonlu olmalı");
    assert!(ci0 <= ci1, "CI alt > üst: {ci0} > {ci1}");
}

// ── Hata durumları ────────────────────────────────────────────────────────────

#[test]
fn ttest_two_without_col2_fails() {
    cmd()
        .args(["ttest", "--kind", "two", "--col", "a"])
        .write_stdin("a,b\n1,2\n3,4\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("col2"));
}

#[test]
fn ttest_paired_without_col2_fails() {
    cmd()
        .args(["ttest", "--kind", "paired", "--col", "a"])
        .write_stdin("a,b\n1,2\n3,4\n")
        .assert()
        .failure();
}

#[test]
fn ttest_insufficient_data_fails() {
    // n=1 → varyans hesaplanamaz
    cmd()
        .args(["ttest", "--kind", "one", "--col", "v", "--mu", "0"])
        .write_stdin("v\n5\n")
        .assert()
        .failure();
}

#[test]
fn ttest_exit_code_zero_on_success() {
    cmd()
        .args(["ttest", "--kind", "one", "--col", "v", "--mu", "0", "--format", "json"])
        .write_stdin("v\n1\n2\n3\n")
        .assert()
        .code(0);
}
