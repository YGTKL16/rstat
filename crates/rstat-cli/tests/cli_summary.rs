use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rstat-cli").unwrap()
}

// ── Başarılı çalışma ──────────────────────────────────────────────────────────

#[test]
fn summary_file_json() {
    let out = cmd()
        .args(["summary", "tests/fixtures/one_col.csv", "--col", "value", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).expect("geçerli JSON olmalı");
    assert_eq!(v["n"], 10);
    // scipy: mean([5.1,4.9,6.2,5.5,5.0,4.8,6.1,5.3,5.7,4.95]) = 5.355
    let mean = v["mean"].as_f64().unwrap();
    assert!((mean - 5.355).abs() < 1e-10, "mean={mean}");
}

#[test]
fn summary_stdin_json() {
    let out = cmd()
        .args(["summary", "--col", "value", "--format", "json"])
        .write_stdin("value\n1\n2\n3\n4\n5\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["n"], 5);
    assert!((v["mean"].as_f64().unwrap() - 3.0).abs() < 1e-10);
    assert!((v["median"].as_f64().unwrap() - 3.0).abs() < 1e-10);
}

#[test]
fn summary_with_na_skips_rows() {
    let out = cmd()
        .args(["summary", "tests/fixtures/with_na.csv", "--col", "value", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // NA ve boş satır atlanır: [5.1, 6.2, 5.5, 4.8] → n=4
    assert_eq!(v["n"], 4);
}

#[test]
fn summary_table_format() {
    cmd()
        .args(["summary", "--col", "value", "--format", "table"])
        .write_stdin("value\n1\n2\n3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("mean"))
        .stdout(predicate::str::contains("std"));
}

#[test]
fn summary_csv_format() {
    let out = cmd()
        .args(["summary", "--col", "value", "--format", "csv"])
        .write_stdin("value\n1\n2\n3\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let s = String::from_utf8(out).unwrap();
    let mut lines = s.lines();
    assert_eq!(lines.next().unwrap(), "n,mean,std,min,q1,median,q3,max");
    assert!(lines.next().unwrap().starts_with("3,"));
}

#[test]
fn summary_col_by_index() {
    // İndeks ile kolon seçimi (0-tabanlı)
    cmd()
        .args(["summary", "--col", "0", "--format", "json"])
        .write_stdin("x\n10\n20\n30\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"n\": 3"));
}

// ── Hata durumları ────────────────────────────────────────────────────────────

#[test]
fn summary_empty_input_fails() {
    cmd()
        .args(["summary", "--col", "value"])
        .write_stdin("value\n")
        .assert()
        .failure();
}

#[test]
fn summary_nonexistent_col_fails() {
    cmd()
        .args(["summary", "--col", "olmayan", "--format", "json"])
        .write_stdin("value\n1\n2\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("olmayan"));
}

#[test]
fn summary_nonexistent_file_fails() {
    cmd()
        .args(["summary", "yok.csv"])
        .assert()
        .failure();
}
